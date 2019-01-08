#!/usr/bin/env python3

import sys, getopt
import csv
import os
import base64
import re
import urllib3
import certifi
import xml.dom.minidom

def parse_fragment(fragment):
    result = dict()
    for param_raw in fragment.split('&'):
        # get the parameter name and value
        p = param_raw.split('=', 1)
        if p[0].startswith('_'):
            # value is base 64 encoded
            if len(p) == 2:
                normalized_value = re.sub('[^a-zA-Z0-9\-_%s]', '', p[1])

                missing_padding = len(normalized_value) % 4
                if missing_padding:
                    normalized_value += '='* (4 - missing_padding)
                result[p[0][1:]] = base64.urlsafe_b64decode(normalized_value).decode("utf-8")
        else:
            if len(p) == 2:
                result[p[0]] = p[1]
    return result
def usage():
    print("usage: " + __file__ + " [-u <url>] [-U username] [-p <password>] <url-shortener-file>")
def main(argv):

    try:
        opts, args = getopt.getopt(argv, "u:U:p:")
        service_url = None
        service_username = None
        service_password = None
        for n,v in opts:
            if n == "-u":
                service_url = v
            elif n == "-U":
                service_username = v
            elif n == "-p":
                service_password = v
    except getopt.GetoptError as err:
        print(err)
        usage()
        sys.exit(1)
    if len(args) < 1:
        print("You have to give the URL shortener tab file as an argument.")
        usage()
        exit(1)

    if service_url is not None:
        http = urllib3.PoolManager(cert_reqs='CERT_REQUIRED', ca_certs=certifi.where())

    fields = ["name", "aql", "corpus", "count"]

    writer = csv.DictWriter(sys.stdout, fieldnames=fields)
    writer.writeheader()

    with open(args[0]) as url_shortener_file:
        idx = 1
        reader = csv.reader(url_shortener_file, delimiter='\t', quoting=csv.QUOTE_NONE, doublequote=False, escapechar=None)
        for row in reader:
            url = row[3]
            # ignore the embedded visualization for now, only use the query references
            if url.startswith('/#'):
                # parse the fragment of the URL
                params = parse_fragment(url[2:])
                if params["q"] is not None and len(params["q"]) > 0:
                    query_def = dict()
                    query_def["name"] = str(idx)
                    query_def["corpus"] = params["c"]
                    query_def["aql"] = params["q"]
                    query_def["count"] = -1

                    # try to query the number of results   
                    if service_url is not None:
                        count_endpoint = service_url + "/annis/query/search/count"
                        headers = None
                        if service_username is not None and service_password is not None:
                            headers = urllib3.make_headers(basic_auth=service_username + ":" + service_password)

                        response = http.request('GET', count_endpoint, {'corpora' : query_def["corpus"], 'q': query_def["aql"]}, headers)
                        if response.status == 200:
                            # Result is an XML
                            result = xml.dom.minidom.parseString(response.data)
                            query_def["count"] = int(result.getElementsByTagName("matchCount")[0].childNodes[0].data)

                    writer.writerow(query_def)
                    sys.stdout.flush()
                    idx += 1



if __name__ == "__main__":
   main(sys.argv[1:])
