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

Query JSONQueryParser::parse(const DB& db, std::istream& jsonStream) {
  Query q(db);
  
  Json::Value parsed;
  jsonStream >> parsed;
   
  return q;
}

JSONQueryParser::~JSONQueryParser() {
}

