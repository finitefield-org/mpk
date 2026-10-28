#!/usr/bin/env python3
"""Apply Clippy and test cfg only to the selected original example target."""
import json
import os
import sys

compiler, *arguments = sys.argv[1:]
positions = [index for index, argument in enumerate(arguments) if argument == '--crate-name']
target = len(positions) == 1 and arguments[positions[0] + 1] == 'go_practical_live_combined_roots'
source = 'examples/go_practical_live_combined_roots.rs'
target = target and any(argument == source or argument.endswith('/' + source) for argument in arguments)
if target:
    driver = os.environ['MPK_SCOPED_CLIPPY_DRIVER']
    log = os.environ['MPK_SCOPED_CLIPPY_LOG']
    with open(log, 'a', encoding='utf-8') as stream:
        stream.write(json.dumps({'compiler': compiler, 'driver': driver, 'arguments': arguments, 'extra_arguments': ['--cfg', 'test', '-Aexplicit_builtin_cfgs_in_flags', '-Dwarnings']}) + '\n')
    os.execv(driver, [driver, compiler, *arguments, '--cfg', 'test', '-Aexplicit_builtin_cfgs_in_flags', '-Dwarnings'])
os.execv(compiler, [compiler, *arguments])
