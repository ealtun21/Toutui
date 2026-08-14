#!/usr/bin/env python3
"""A proxy that gives a part of a stream of HLS that holds no audio.

    python3 docs/harness/a_part_that_holds_no_audio.py 13509 13399 \\
        requests.log 1

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, and the number of the first part that holds no audio. A request of
`output-N.ts` with N of that number or more receives a body of packets of
padding: 32 packets of 188 bytes, each of the identity 0x1FFF. Every other
request goes to the sandbox and comes back whole.

**A body of padding is a whole body.** The rule of T-194 says that a body whose
length is no whole number of packets of 188 bytes is a body that stopped;
this body holds 6016 bytes, therefore that rule says nothing of it. The body is a
transport stream of the form of the container, and it holds no packet of the
identity of the audio: `audio_payload` of the program gives no byte of it.

**Such a part comes of a server that started its ffmpeg again.** ffmpeg of
Audiobookshelf writes the parts while the client reads them, and it starts again
with `-c:a aac` when the first try dies (T-68). The identity of the audio of the
new parts belongs to the new ffmpeg, and the reader of the program holds the
identity of the first part alone. See T-195.

The address of this proxy goes in `users.server_address` of `db.sqlite3` of the
sandbox, and **the pool holds one address alone**: a second address takes the
request that this proxy answers (T-97, and the trap 129).
"""
import asyncio
import re
import sys

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
LOG = sys.argv[3]
FROM = int(sys.argv[4])

END_OF_THE_HEAD = b"\r\n\r\n"
THE_NAME_OF_A_PART = re.compile(r"/output-(\d+)\.ts")

PACKET = 188
PACKETS_OF_THE_BODY = 32


def a_packet_of_padding(number):
    """Gives one packet of 188 bytes of the identity 0x1FFF and no audio."""
    head = bytes([0x47, 0x1F, 0xFF, 0x10 | (number % 16)])
    return head + bytes([0xFF] * (PACKET - len(head)))


THE_BODY_OF_NO_AUDIO = b"".join(
    a_packet_of_padding(number) for number in range(PACKETS_OF_THE_BODY)
)


def say(words):
    with open(LOG, "a") as log:
        log.write(words + "\n")


def holds_no_audio(path):
    """Tells if this path is a part that the proxy answers itself."""
    found = THE_NAME_OF_A_PART.search(path)
    return found is not None and int(found.group(1)) >= FROM


async def read_the_head(reader):
    """Reads bytes until the end of the head of a message of HTTP."""
    buffer = b""
    while END_OF_THE_HEAD not in buffer:
        block = await reader.read(4096)
        if not block:
            return buffer, b""
        buffer += block
    head, rest = buffer.split(END_OF_THE_HEAD, 1)
    return head + END_OF_THE_HEAD, rest


def with_the_close(head):
    """Gives the head of a request that asks for a connection of its own."""
    lines = head.split(b"\r\n")
    kept = [
        line
        for line in lines
        if not line.lower().startswith(b"connection:")
        and not line.lower().startswith(b"keep-alive:")
    ]
    while kept and kept[-1] == b"":
        kept.pop()
    kept.append(b"Connection: close")
    return b"\r\n".join(kept) + END_OF_THE_HEAD


async def to_the_server(reader, writer, rest):
    if rest:
        writer.write(rest)
        await writer.drain()
    while True:
        block = await reader.read(65536)
        if not block:
            break
        writer.write(block)
        await writer.drain()
    try:
        writer.write_eof()
    except OSError:
        pass


async def to_the_client(reader, writer):
    while True:
        block = await reader.read(65536)
        if not block:
            return
        writer.write(block)
        await writer.drain()


async def the_answer_of_no_audio(writer):
    head = (
        b"HTTP/1.1 200 OK\r\n"
        b"Content-Type: video/mp2t\r\n"
        b"Content-Length: %d\r\n"
        b"Connection: close\r\n\r\n" % len(THE_BODY_OF_NO_AUDIO)
    )
    writer.write(head + THE_BODY_OF_NO_AUDIO)
    await writer.drain()


async def one_connection(reader, writer):
    head, rest = await read_the_head(reader)
    if not head:
        writer.close()
        return

    line = head.split(b"\r\n", 1)[0].decode("latin-1")
    parts = line.split(" ")
    path = parts[1] if len(parts) > 1 else ""

    if holds_no_audio(path):
        say("%s  [the part holds no audio]" % line)
        try:
            await the_answer_of_no_audio(writer)
        except OSError:
            pass
        try:
            writer.close()
        except OSError:
            pass
        return

    say(line)

    try:
        to_reader, to_writer = await asyncio.open_connection("127.0.0.1", TARGET)
    except OSError as error:
        say("no connection to the sandbox: %s" % error)
        writer.close()
        return

    to_writer.write(with_the_close(head))
    await to_writer.drain()

    the_road_to_the_server = asyncio.ensure_future(to_the_server(reader, to_writer, rest))
    try:
        await to_the_client(to_reader, writer)
    except (OSError, asyncio.IncompleteReadError):
        pass
    the_road_to_the_server.cancel()

    for each in (writer, to_writer):
        try:
            each.close()
        except OSError:
            pass


async def main():
    server = await asyncio.start_server(one_connection, "127.0.0.1", PORT)
    say(
        "the proxy of a part that holds no audio holds the port %d, from the part %d"
        % (PORT, FROM)
    )
    async with server:
        await server.serve_forever()


asyncio.run(main())
