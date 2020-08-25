#!/bin/bash

if [[ "$TRAVIS_OS_NAME" == "linux" ]]
then 
    mv target/release/libgraphannis_capi.so target/release/libgraphannis.so
fi
if [[ "$TRAVIS_OS_NAME" == "osx" ]]
then 
    mv target/release/libgraphannis_capi.dylib target/release/libgraphannis.dylib
    mv target/release/annis target/release/annis.osx
    mv target/release/graphannis-webservice target/release/graphannis-webservice.osx  
fi
if [[ "$TRAVIS_OS_NAME" == "windows" ]]
then 
     mv target/release/graphannis_capi.dll target/release/graphannis.dll
fi