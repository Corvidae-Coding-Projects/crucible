#!/bin/sh
# The fixture models a detected memory-safety violation as an abnormal target termination.
kill -ABRT $$
