#!/bin/sh
curl -L -o gum.zip https://github.com/amir-zeldes/gum/archive/f8ac9944fae39ae37c4db186304e3c1ab41f77f3.zip 
unzip -o -j gum.zip "gum-*/annis/GUM_annis/*" -d relannis/GUM
