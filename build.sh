#!/bin/sh

zip -r -o -X -ll my module-$(cat module.prop | grep 'version=' | awk -F '=' '{print $2}').zip ./my module
