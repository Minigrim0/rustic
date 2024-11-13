import pytest
import rustic


def test_sum_as_string():
    assert rustic.sum_as_string(1, 1) == "2"
