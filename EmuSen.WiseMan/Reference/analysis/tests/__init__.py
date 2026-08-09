"""Makes `analysis` importable from the tests however discovery was invoked.

    cd EmuSen.WiseMan/Reference/analysis && python3 -m unittest discover -s tests

No venv and no install step - see EmuSen_Stack.md §2.2.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
