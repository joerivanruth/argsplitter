#!/usr/bin/env python3

import subprocess
from subprocess import PIPE
from typing import Optional

README_FILE = 'README.md'
DEMO_FILE = 'examples/send_mail.rs'


class Cursor:
    cur: Optional[str]
    before: list[str]
    after: list[str]

    def __str__(self):
        return f"Cursor({self.before!r} / {self.cur!r} / {self.after!r}"

    def __init__(self, lines: list[str]):
        self.cur = None
        self.before = []
        self.after = [line.rstrip() for line in lines]
        self.next()

    def rewind(self):
        self.after = self.all()
        self.before = []
        self.cur = None
        self.next()

    def next(self):
        if self.cur is not None:
            self.before.append(self.cur)
        if self.after:
            self.cur = self.after[0]
            self.after = self.after[1:]
        else:
            self.cur = None

    def insert_after(self, lines: list[str]):
        self.after = lines[:] + self.after

    def insert_before(self, lines: list[str]):
        self.before = self.before + lines[:]

    def remove(self):
        self.cur = None
        c.next()

    def all(self):
        return self.before[:] + ([self.cur] if self.cur is not None else []) + self.after[:]


def run_example(cmd: str):
    s = ""
    parts = cmd.split(' ', 2)
    exe = parts[0]
    args = ' '.join(parts[1:]) if len(parts) > 1 else ""
    cmd = f"cargo run -q --example={exe} -- {args}"
    # s += f"-- cmd {cmd!r} --\n"
    p = subprocess.run(cmd, shell=True, stderr=PIPE, stdout=PIPE)

    if p.stdout:
        s += "-- stdout --\n" + str(p.stdout, 'utf-8')
    if p.stderr:
        s += "-- stderr --\n" + str(p.stderr, 'utf-8')
    if p.returncode != 0:
        s += f"-- exit status {p.returncode}"

    return s.splitlines()

c = Cursor(open(README_FILE).readlines())

while c.cur is not None:
    if c.cur.strip() == "```rust":
        c.next()
        c.insert_before([line.rstrip() for line in open(DEMO_FILE).readlines()])
        while not c.cur.startswith("```"):
            c.remove()
        c.next()
    if c.cur.startswith("» "):
        cmd = c.cur[2:].strip()
        c.next()
        while not c.cur.startswith("```"):
            c.remove()
        # c.insert_before(['banana\n', 'phone\n'])
        c.insert_before(run_example(cmd))

    c.next()

with open(README_FILE, 'w') as f:
    for line in c.all():
        print(line, file=f)
