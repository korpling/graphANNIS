#!/usr/bin/env python3

from distutils.core import setup

setup(name='graphannis',
      version='1.0',
      description='graphANNIS Python API',
      author='Thomas Krause',
      author_email='thomaskrause@posteo.de',
      url='https://github.com/thomaskrause/graphANNIS/',
      packages=['graphannis'],
      package_data={'graphannis' : ['../target/release/libgraphannis_capi.so']},
     )
