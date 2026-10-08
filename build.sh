#!/bin/bash

cargo b -r && makepkg -fc && makepkg -f --nodeps --nocheck
