# -*- coding: utf-8 -*-
"""
Created on Wed Dec 14 08:53:14 2016

@author: thomas
"""

import pandas as pd
import matplotlib.pyplot as plt
import collections
import numpy as np

import json


def getaql(x, querydir):
    aqlFileName = ""
    group = x.Group
    problemSpace = x.queryID

    aqlFileName = querydir + "/" + group + "/" + str.format("{:05d}", problemSpace) + ".aql"

    with open (aqlFileName, "r") as aqlFile:
        aql = aqlFile.read()
        
    return aql
    
    
def get_num_of_nodes(x, querydir):
    jsonFileName = ""
    group = x.Group
    problemSpace = x.queryID

    jsonFileName = querydir + "/" + group + "/" + str.format("{:05d}", problemSpace) + ".json"

    with open (jsonFileName, "r") as jsonFile:
        jsonContent = json.loads(jsonFile.read())
        
    return len(jsonContent["alternatives"][0]["nodes"])    

def extract(fn, querydir=None):
    data = pd.read_csv(fn, delim_whitespace=False)
    
    # rename columns that have python incompatible names
    data.rename(columns={"us/Iteration": "time", "Problem Space" : "queryID"}, inplace=True)
    
    # don't use microseconds but milliseconds
    data.time = data.time / 1000.0

    if querydir is not None:
        # try to get the original AQL queries
        data['aql'] = data.apply(getaql, args=(querydir,), axis=1)
        data['numofnodes'] = data.apply(get_num_of_nodes, args=(querydir,), axis=1)
    else:
        # add empty query as column to data
        data['aql'] = pd.Series("", index=data.index)
    return data
    
def desc(d):

    Desc = collections.namedtuple("Desc", ["worse", "better", "quantile", "sumTime"])    
    
    worse = len(d.loc[d.Baseline >= 1.0])
    better = len(d.loc[d.Baseline < 1.0])
    
    q = d.Baseline.quantile([.1, .25, .5, .75, 1.0])    

    sumTime = d.time.sum()    
    
    return Desc(worse, better, q, sumTime)
    
def plot(d, header=None):

    h = (1.0 / d.Baseline).sort_values().to_frame()    
    h["index1"] = range(len(h))
    h.columns = ["speedup", "aql"]


    fig = plt.figure()
    ax = fig.gca()
    ax.get_yaxis().set_label_text("times faster than baseline")
    ax.set_yscale('log')    
    
    ax.get_xaxis().set_label_text("query")
    ax.get_xaxis().set_visible(False)
    
    ax.set_xlim([-0.01*len(h),len(h)*1.01])
    
    plt.scatter(x=h.aql, y=h.speedup, marker="*")
    plt.grid(True)

    plt.axhline(y=1.0, xmin=0, xmax=1, hold=None, color="#FF0000")
    
    plt.show()


