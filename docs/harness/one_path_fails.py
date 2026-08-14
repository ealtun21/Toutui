#!/usr/bin/env python3
"""A proxy that gives one fault to some requests, and that forwards the others.

**A server that answers some requests and that fails others** is a condition
that no other harness of this repository gives. `slow.py` gives a delay to
every request, and `blackhole.py` takes the whole server away: each of them
puts the program in the offline mode of T-25, and the words of that mode are
right there. A server that answers the libraries and that fails the
collections and the playlists is a different condition, and the program must
say the truth of it.

    python3 docs/harness/one_path_fails.py 13500 13399 requests.log /collections /playlists

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, and then one or more parts of a path. A request whose path holds one of
those parts takes the answer `500`, and every other request goes to the sandbox.

The pool of the program takes this address with a block `[[servers]]` of
`config.toml`, as `slow.py` does:

    [[servers]]
    name = "sandbox"
    endpoints = [
      { url = "http://127.0.0.1:13500", priority = 0 },
      { url = "http://localhost:13399", priority = 1 },
    ]

**The answer `500` is an answer**, therefore the address keeps the state `Up`
(T-128) and every request of the program stays with this proxy.

This proxy reads one request after the other on a connection, and it opens one
connection of its own to the sandbox for each of them. It is made for the start
of the program and for the keys of the views. It is not made for a stream of
audio.
"""
import asyncio
import sys
import time

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
LOG = open(sys.argv[3], "w", buffering=1)
PARTS = sys.argv[4:]
START = time.monotonic()

THE_FAULT = (
    b"HTTP/1.1 500 Internal Server Error\r\n"
    b"Content-Type: text/plain\r\n"
    b"Content-Length: 21\r\n"
    b"Connection: keep-alive\r\n"
    b"\r\n"
    b"this proxy said no.\r\n"
)


def note(text):
    LOG.write("%8.3f %s\n" % (time.monotonic() - START, text))


async def one_request(reader):
    """Reads one request. It gives the head, the body, and the path."""
    head = await reader.readuntil(b"\r\n\r\n")
    lines = head.split(b"\r\n")
    path = lines[0].split(b" ")[1].decode("latin1") if len(lines[0].split(b" ")) > 1 else ""

    length = 0
    for line in lines[1:]:
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1].strip())

    body = await reader.readexactly(length) if length else b""
    return head, body, path


async def to_the_sandbox(head, body):
    """Sends one request to the sandbox, and it gives the whole answer."""
    reader, writer = await asyncio.open_connection("127.0.0.1", TARGET)
    head = head.replace(b"Connection: keep-alive\r\n", b"")
    head = head[:-2] + b"Connection: close\r\n\r\n"
    writer.write(head + body)
    await writer.drain()
    answer = await reader.read()
    writer.close()
    return answer


async def one_connection(client_reader, client_writer):
    try:
        while True:
            head, body, path = await one_request(client_reader)

            if any(part in path for part in PARTS):
                note("500 %s" % path)
                client_writer.write(THE_FAULT)
                await client_writer.drain()
                continue

            note("--- %s" % path)
            answer = await to_the_sandbox(head, body)
            # The answer of the sandbox says `Connection: close`, therefore this
            # connection ends with it.
            client_writer.write(answer)
            await client_writer.drain()
            break
    except (asyncio.IncompleteReadError, ConnectionResetError, BrokenPipeError):
        pass
    finally:
        client_writer.close()


async def main():
    server = await asyncio.start_server(one_connection, "127.0.0.1", PORT)
    note("the proxy holds the port %d, and it fails %s" % (PORT, " ".join(PARTS)))
    async with server:
        await server.serve_forever()


asyncio.run(main())
