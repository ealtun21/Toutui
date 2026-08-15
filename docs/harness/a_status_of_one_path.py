#!/usr/bin/env python3
"""A proxy that gives one status of the command line to some requests.

`one_path_fails.py` answers the status `500` alone, and **a status of 401 is
not a status of 500**: the program reads 401 as a token that the server refuses
(T-123), and it then takes the road of a new login. No other harness of this
repository gives that road with the sandbox behind it, and a token of the
database that a `sqlite3` writes gives `Failed to decrypt the token.` (the trap
192).

    python3 docs/harness/a_status_of_one_path.py 13510 13399 requests.log 401 /api/libraries

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, the status, and then one or more parts of a path. A request whose path
holds one of those parts takes that status, and every other request goes to the
sandbox.

The account takes this address in `users.server_address` of the database of the
program (the trap 129), and a copy of that file gives the address of the
sandbox back at the end.

**A status of the 4xx and of the 5xx is an answer**, therefore the address keeps
the state `Up` (T-128) and every request of the program stays with this proxy.

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
STATUS = int(sys.argv[4])
PARTS = sys.argv[5:]
START = time.monotonic()

THE_FAULT = (
    ("HTTP/1.1 %d This proxy answers\r\n" % STATUS).encode("latin1")
    + b"Content-Type: text/plain\r\n"
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
                note("%d %s" % (STATUS, path))
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
