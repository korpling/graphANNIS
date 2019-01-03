#!/bin/sh
curl -L -o gum.zip https://github.com/amir-zeldes/gum/archive/f8ac9944fae39ae37c4db186304e3c1ab41f77f3.zip 
unzip -o -j gum.zip "gum-*/annis/GUM_annis/*" -d relannis/GUM

curl -L -o pcc21.zip http://angcl.ling.uni-potsdam.de/resources/pcc2.1_annis.zip
unzip -o -j pcc21.zip "pcc2.1/annis/*" -d relannis/pcc2.1

curl -L -o ridges7.zip https://www.linguistik.hu-berlin.de/de/institut/professuren/korpuslinguistik/forschung/ridges-projekt/download-files/v7/annis.zip
unzip -o -j ridges7.zip "Annis/*" -d relannis/RIDGES_Herbology_Version7.0
