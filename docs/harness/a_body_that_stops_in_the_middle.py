#!/usr/bin/env python3
"""A proxy that stops the body of one answer in the middle.

    python3 docs/harness/a_body_that_stops_in_the_middle.py 13507 13399 \\
        requests.log 60000 /ebook

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, the number of the bytes of the body that the client receives, and the parts
of a path. A request whose path holds one of those parts receives the head of
the answer and the first bytes of the body alone, and the proxy then closes the
connection. Every other request goes to the sandbox and comes back whole.

**A download that stops in the middle is not a download that failed at its
start.** `one_path_fails.py` gives the status 500 before the first byte, and a
program that writes the answer to a file then writes nothing at all. This proxy
gives the fault of the network of the real world: the head says the length of
the whole file, the bytes stop, and the file of the disk then holds a part of a
book with the name of a whole book. See T-186.

**The proxy gives every request a connection of its own.** It writes
`Connection: close` in the head of the request, therefore the sandbox closes
after each answer and no connection of the pool of `reqwest` holds the fault of
a truncation for the request after it (the trap 145). One connection then holds
one request, and the decision of the truncation belongs to that request alone.

The address of this proxy goes in `users.server_address` of `db.sqlite3` of the
sandbox, and **the pool holds one address alone**: a second address takes the
request that this proxy breaks (T-97, and the trap 129).
"""
import asyncio
import sys

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
LOG = sys.argv[3]
BYTES_OF_THE_BODY = int(sys.argv[4])
PARTS = sys.argv[5:]

END_OF_THE_HEAD = b"\r\n\r\n"


def say(words):
    with open(LOG, "a") as log:
        log.write(words + "\n")


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
    # The last two parts of the split are the empty lines of the end of the head.
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


async def to_the_client(reader, writer, truncates):
    head, rest = await read_the_head(reader)
    if not head:
        return
    writer.write(head)
    await writer.drain()

    sent = 0
    block = rest
    while True:
        if block:
            if truncates and sent + len(block) >= BYTES_OF_THE_BODY:
                writer.write(block[: BYTES_OF_THE_BODY - sent])
                await writer.drain()
                say("the body stopped after %d bytes" % BYTES_OF_THE_BODY)
                return
            writer.write(block)
            await writer.drain()
            sent += len(block)
        block = await reader.read(65536)
        if not block:
            return


async def one_connection(reader, writer):
    head, rest = await read_the_head(reader)
    if not head:
        writer.close()
        return

    line = head.split(b"\r\n", 1)[0].decode("latin-1")
    path = line.split(" ")[1] if len(line.split(" ")) > 1 else ""
    truncates = any(part in path for part in PARTS)
    say("%s%s" % (line, "  [the body stops]" if truncates else ""))

    try:
        to_reader, to_writer = await asyncio.open_connection("127.0.0.1", TARGET)
    except OSError as error:
        say("no connection to the sandbox: %s" % error)
        writer.close()
        return

    to_writer.write(with_the_close(head))
    await to_writer.drain()

    # **The road of the client stays a task, and the answer decides the end.**
    # A `gather` of the two roads waits for the end of the stream of the client,
    # and a client that waits for the rest of a body sends no end: the proxy
    # then holds that connection for ever.
    the_road_to_the_server = asyncio.ensure_future(to_the_server(reader, to_writer, rest))
    try:
        await to_the_client(to_reader, writer, truncates)
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
    say("the proxy of the body that stops holds the port %d" % PORT)
    async with server:
        await server.serve_forever()


asyncio.run(main())
