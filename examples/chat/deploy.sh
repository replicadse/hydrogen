#!/bin/bash
faas-cli build -f fn-message.yml && kind load docker-image fn-message:latest && faas-cli deploy -f fn-message.yml
