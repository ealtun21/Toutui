#!/usr/bin/env python3
"""A proxy that answers `GET /api/libraries` with a body of a file.

`no_library.py` gives one body — the empty list of T-173 — and the answer of
that endpoint holds more shapes that a server can give: a library that holds no
name, a body that is not the JSON of that endpoint, and a body of no JSON at
all. **The program reads that answer at the login and at every start**, and
`Root` of `src/api/libraries/get_all_libraries.rs` asks for every field of it.

    python3 docs/harness/another_body_of_the_libraries.py 13502 13399 \
        requests.log /the/absolute/path/of/the/body.json

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, and the file of the body. Every other request goes to the sandbox,
therefore the login of the sandbox works through this proxy and the token is a
real token.

**Give the absolute path of the two files** (the trap 132), and give the account
one address alone (the trap 129).
"""
import asyncio
import sys
import time

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
LOG = open(sys.argv[3], "w", buffering=1)
THE_BODY = open(sys.argv[4], "rb").read()
START = time.monotonic()

THE_ANSWER = (
    b"HTTP/1.1 200 OK\r\n"
    b"Content-Type: application/json\r\n"
    b"Content-Length: %d\r\n"
    b"Connection: close\r\n"
    b"\r\n" % len(THE_BODY)
) + THE_BODY


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

            # `/api/libraries/:id` is the library of an id, and this proxy
            # leaves it alone.
            if path.split("?")[0].rstrip("/") == "/api/libraries":
                note("<< %s" % path)
                client_writer.write(THE_ANSWER)
                await client_writer.drain()
                break

            note("-- %s" % path)
            answer = await to_the_sandbox(head, body)
            client_writer.write(answer)
            await client_writer.drain()
            break
    except (asyncio.IncompleteReadError, ConnectionResetError, BrokenPipeError):
        pass
    finally:
        client_writer.close()


async def main():
    server = await asyncio.start_server(one_connection, "127.0.0.1", PORT)
    note("the proxy holds the port %d, and it gives %d bytes" % (PORT, len(THE_BODY)))
    async with server:
        await server.serve_forever()


asyncio.run(main())
