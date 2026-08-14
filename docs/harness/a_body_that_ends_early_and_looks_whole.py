#!/usr/bin/env python3
"""A proxy that gives a body that stops in the middle and that looks whole.

    python3 docs/harness/a_body_that_ends_early_and_looks_whole.py 13508 13399 \\
        requests.log 20000 /api/items /hls

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, the number of the bytes of the body that the client receives, and the parts
of a path. A request whose path holds one of those parts receives the head of
the answer and the first bytes of the body alone, and the proxy then closes the
connection. Every other request goes to the sandbox and comes back whole.

**This proxy is not `a_body_that_stops_in_the_middle.py`.** That one keeps the
head of the sandbox, and that head holds `Content-Length`: a client of `reqwest`
then counts the bytes, it finds fewer of them, and it gives the fault of an
incomplete message. The program reads a fault, and it can say it.

This proxy takes `Content-Length` and `Transfer-Encoding` **out** of the head of
the answer, and it writes `Connection: close` in their place. A body with
neither of those headers ends at the close of the connection (RFC 9112, section
6.3), therefore the client reads a **clean** end of the body: no fault of the
network, no fault of a status, and fewer bytes than the file holds. This is the
answer that a proxy in front of Audiobookshelf gives when it loses its own
connection to the server, and it is the one condition where a program can hold a
part of a book for the whole book with no word of a fault at all. See T-193.

**The head of the answer keeps `Content-Range`.** That header names the size of
the whole file, and a client that reads it holds the one truth of the length:
the number of the bytes that the file has. A body that stops before that number
is a connection that stopped.

**The proxy gives every request a connection of its own.** It writes
`Connection: close` in the head of the request too, therefore the sandbox closes
after each answer and no connection of the pool of `reqwest` holds the fault of
a truncation for the request after it (the trap 145).

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
    """Gives a head that asks for a connection of its own."""
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


def with_no_length(head):
    """Gives a head of an answer whose body ends at the close.

    `Content-Length` and `Transfer-Encoding` each name the end of a body. A
    head that holds neither of them gives a body that ends at the close of the
    connection, and the client then reads a clean end of that body.
    """
    lines = head.split(b"\r\n")
    kept = [
        line
        for line in lines
        if not line.lower().startswith(b"content-length:")
        and not line.lower().startswith(b"transfer-encoding:")
        and not line.lower().startswith(b"connection:")
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


async def to_the_client(reader, writer, truncates):
    head, rest = await read_the_head(reader)
    if not head:
        return

    writer.write(with_no_length(head) if truncates else head)
    await writer.drain()

    if not truncates:
        block = rest
        while True:
            if block:
                writer.write(block)
                await writer.drain()
            block = await reader.read(65536)
            if not block:
                return

    sent = 0
    block = rest
    while True:
        if block:
            if sent + len(block) >= BYTES_OF_THE_BODY:
                writer.write(block[: BYTES_OF_THE_BODY - sent])
                await writer.drain()
                say(
                    "the body stopped after %d bytes, and the head named no length"
                    % BYTES_OF_THE_BODY
                )
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
    say("%s%s" % (line, "  [the body ends early]" if truncates else ""))

    try:
        to_reader, to_writer = await asyncio.open_connection("127.0.0.1", TARGET)
    except OSError as error:
        say("no connection to the sandbox: %s" % error)
        writer.close()
        return

    to_writer.write(with_the_close(head))
    await to_writer.drain()

    # **The road of the client stays a task, and the answer decides the end.**
    # A client that waits for the rest of a body sends no end (the trap 146).
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
    say("the proxy of the body that looks whole holds the port %d" % PORT)
    async with server:
        await server.serve_forever()


asyncio.run(main())
