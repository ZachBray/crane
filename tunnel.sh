#!/bin/sh

sudo apt install -y autossh
autossh -M 0 -R craneci:80:localhost:8080 serveo.net
