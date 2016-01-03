/* 
 * File:   JSONQueryParser.cpp
 * Author: Thomas Krause <thomaskrause@posteo.de>
 * 
 * Created on 3. Januar 2016, 16:05
 */

#include <json/json.h>
#include "jsonqueryparser.h"

using namespace annis;

JSONQueryParser::JSONQueryParser() {
}

Query JSONQueryParser::parse(const DB& db, const std::string& queryAsJson) {
  Query q(db);
  
  Json::Value parsed;
  std::cin >> parsed;
   
  return q;
}

JSONQueryParser::~JSONQueryParser() {
}

