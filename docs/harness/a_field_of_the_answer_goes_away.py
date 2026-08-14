#!/usr/bin/env python3
"""A proxy that takes one field away from the answer of a path.

`another_body_of_the_libraries.py` gives the whole body of one endpoint from a
file (T-176), and a body of a file cannot hold a token or an id that the sandbox
made at the moment of the request. **`GET /api/me` holds such values**: the id
of the account, the id of every media, and the position of each of them.
Therefore this proxy forwards the request to the sandbox, and it takes the named
fields out of the answer of that one path.

    python3 docs/harness/a_field_of_the_answer_goes_away.py 13503 13399 \
        requests.log /api/me mediaItemId mediaItemType

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, the path, and one name of a field or more. Every field of that name goes
away at every depth of the body, therefore a field of a row of a list goes away
in every row. Every other request goes to the sandbox, therefore the login works
through this proxy and the token is a real token.

**A server of another version is the measurement.** `mediaItemId` and
`mediaItemType` came to `mediaProgress` of Audiobookshelf with the version
2.5.0, and a server before that one holds no such field.

**Give the absolute path of the file of the log** (the trap 132), and give the
account one address alone (the trap 129).
"""
import asyncio
import json
import sys
import time

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
LOG = open(sys.argv[3], "w", buffering=1)
THE_PATH = sys.argv[4]
THE_FIELDS = set(sys.argv[5:])
START = time.monotonic()


def note(text):
    LOG.write("%8.3f %s\n" % (time.monotonic() - START, text))


def without_the_fields(value):
    """Gives the value again, and every named field of it goes away."""
    if isinstance(value, dict):
        return {
            key: without_the_fields(one)
            for key, one in value.items()
            if key not in THE_FIELDS
        }
    if isinstance(value, list):
        return [without_the_fields(one) for one in value]
    return value


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


def the_body_of_the_answer(answer):
    """Gives the head and the body of an answer. The body of a chunked answer
    comes together here, because the answer of this proxy holds a length."""
    head, _, body = answer.partition(b"\r\n\r\n")
    if b"chunked" not in head.lower():
        return head, body

    whole = b""
    while True:
        line, _, body = body.partition(b"\r\n")
        size = int(line.split(b";")[0], 16)
        if size == 0:
            break
        whole += body[:size]
        body = body[size + 2 :]
    return head, whole


def the_answer_again(head, body):
    """Makes an answer of a head and of a body, with the length of that body."""
    lines = [
        line
        for line in head.split(b"\r\n")
        if not line.lower().startswith(b"content-length:")
        and not line.lower().startswith(b"transfer-encoding:")
        and not line.lower().startswith(b"connection:")
    ]
    lines.append(b"Content-Length: %d" % len(body))
    lines.append(b"Connection: close")
    return b"\r\n".join(lines) + b"\r\n\r\n" + body


async def one_connection(client_reader, client_writer):
    try:
        while True:
            head, body, path = await one_request(client_reader)

            answer = await to_the_sandbox(head, body)

            if path.split("?")[0].rstrip("/") == THE_PATH.rstrip("/"):
                the_head, the_body = the_body_of_the_answer(answer)
                try:
                    value = json.loads(the_body)
                except ValueError:
                    note("!! %s: the answer holds no JSON" % path)
                else:
                    the_body = json.dumps(without_the_fields(value)).encode()
                    answer = the_answer_again(the_head, the_body)
                    note("<< %s: %s goes away" % (path, ", ".join(sorted(THE_FIELDS))))
            else:
                note("-- %s" % path)

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
        "the proxy holds the port %d, and %s goes away of %s"
        % (PORT, ", ".join(sorted(THE_FIELDS)), THE_PATH)
    )
    async with server:
        await server.serve_forever()


asyncio.run(main())
