#!/bin/bash
rg "^//!" src/lib.rs | sed "s#//!##" >README.md