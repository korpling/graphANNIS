#!/bin/sh
curl -L -o gum.zip https://github.com/amir-zeldes/gum/archive/f8ac9944fae39ae37c4db186304e3c1ab41f77f3.zip 
unzip -o -j gum.zip "gum-*/annis/GUM_annis/*" -d relannis/GUM

curl -L -o pcc21.zip http://angcl.ling.uni-potsdam.de/resources/pcc2.1_annis.zip
unzip -o -j pcc21.zip "pcc2.1/annis/*" -d relannis/pcc2.1

curl -L -o subtokdemo.zip https://corpus-tools.org/corpora/subtok.demo_relANNIS.zip
unzip -o -j subtokdemo.zip "*" -d relannis/subtok.demo
