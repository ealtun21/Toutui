#!/usr/bin/env python3
"""A proxy that gives a status of its own to each part of a path.

`one_path_fails.py` gives the status 500 to every rule, and
`a_status_of_one_path.py` gives one status of the command line to every rule.
**A road of the program that asks the server two times needs two statuses
together**: the reader of T-279 asks for the ebook, and a status 404 of that
request alone takes it to a second request of the item — a measurement of the
answer of that second request therefore needs 404 of the first path and another
status of the second one.

    python3 docs/harness/a_status_of_some_paths.py 13511 13399 requests.log \
        404:/api/items/<the id>/ebook 500:/api/items/<the id>

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, and then one or more rules of the shape `STATUS:part-of-a-path`. **The
first rule that holds a part of the path wins**, therefore a rule of a long path
belongs before a rule of a path that is a part of it. Every other request goes
to the sandbox.

**Two proxies of this repository do not stand one behind the other**: the
answer of a rule says `Connection: keep-alive`, and `to_the_sandbox` of a proxy
in front of it then waits for an end of the stream that never comes. This
harness gives the two statuses of one proxy, and that road has no wait at all.

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
RULES = [(int(rule.split(":", 1)[0]), rule.split(":", 1)[1]) for rule in sys.argv[4:]]
START = time.monotonic()

THE_NAMES = {
    400: "Bad Request",
    401: "Unauthorized",
    403: "Forbidden",
    404: "Not Found",
    500: "Internal Server Error",
    502: "Bad Gateway",
    503: "Service Unavailable",
}


def the_answer(status):
    """Gives the whole answer of one status, with a body of its own."""
    body = b"this proxy said %d.\r\n" % status
    head = (
        "HTTP/1.1 %d %s\r\n"
        "Content-Type: text/plain\r\n"
        "Content-Length: %d\r\n"
        "Connection: keep-alive\r\n"
        "\r\n" % (status, THE_NAMES.get(status, "Fault"), len(body))
    )
    return head.encode("latin1") + body


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


def the_status_of(path):
    """Gives the status of the first rule that holds a part of the path."""
    for status, part in RULES:
        if part in path:
            return status
    return None


async def one_connection(client_reader, client_writer):
    try:
        while True:
            head, body, path = await one_request(client_reader)
            status = the_status_of(path)

            if status is not None:
                note("%d %s" % (status, path))
                client_writer.write(the_answer(status))
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
    note(
        "the proxy holds the port %d, and its rules are %s"
        % (PORT, " ".join("%d:%s" % rule for rule in RULES))
    )
    async with server:
        await server.serve_forever()


asyncio.run(main())
