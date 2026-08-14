#!/usr/bin/env python3
"""A proxy that gives an empty list of the libraries, and that forwards the rest.

**A server that holds no library that the account can reach** is a condition
that no other harness of this repository gives. `one_path_fails.py` gives the
status 500 to a path, and a fault is not this condition: this server answers
`GET /api/libraries` with the status 200 and the body `{"libraries": []}`. A
new Audiobookshelf server before the first library gives that answer, and an
account of no library gives it too.

    python3 docs/harness/no_library.py 13501 13399 requests.log

The arguments: the port of this proxy, the port of the sandbox, and the file of
the log. The login of the sandbox works through this proxy, therefore the token
is a real token of the sandbox.

**Give the absolute path of this file to a command that holds a `cd`** (the
trap 132).

This proxy reads one request after the other on a connection, and it opens one
connection of its own to the sandbox for each of them. It is made for the login
and for the start of the program. It is not made for a stream of audio.
"""
import asyncio
import sys
import time

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
LOG = open(sys.argv[3], "w", buffering=1)
START = time.monotonic()

THE_EMPTY_LIST = (
    b'HTTP/1.1 200 OK\r\n'
    b'Content-Type: application/json\r\n'
    b'Content-Length: 17\r\n'
    b'Connection: close\r\n'
    b'\r\n'
    b'{"libraries": []}'
)


def note(text):
    LOG.write("%8.3f %s\n" % (time.monotonic() - START, text))


async def one_request(reader):
    """Reads one request. It gives the head, the body, and the path."""
    head = await reader.readuntil(b"\r\n\r\n")
    lines = head.split(b"\r\n")
    first = lines[0].split(b" ")
    path = first[1].decode("latin1") if len(first) > 1 else ""

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

            # The path of the libraries holds no other part: `/api/libraries/:id`
            # is the library of an id, and this proxy leaves it alone.
            if path.split("?")[0].rstrip("/") == "/api/libraries":
                note("[] %s" % path)
                client_writer.write(THE_EMPTY_LIST)
                await client_writer.drain()
                break

            note("-- %s" % path)
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
    note("the proxy holds the port %d, and it gives no library" % PORT)
    async with server:
        await server.serve_forever()


asyncio.run(main())
