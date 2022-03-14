#!/usr/bin/env python
# -*- coding: utf-8 -*-

import sys
import os

if __name__ == "__main__":
    script_dir = os.path.dirname(__file__)
    sys.path.append(script_dir)
    from xresconv_cli import main

    main()
