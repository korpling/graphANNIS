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
  
  // parse root as value
  Json::Value root;
  jsonStream >> root;
  
  // get the first alternative (we don't support more than one currently)
  const auto& alternatives = root["alternatives"];
  if(alternatives.size() != 0) {
    const auto& firstAlt = alternatives[0];
    
    // add all nodes
    const auto& nodes = firstAlt["nodes"];
    
    
  }
  return q;
}

JSONQueryParser::~JSONQueryParser() {
}

