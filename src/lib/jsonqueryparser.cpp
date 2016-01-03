/* 
 * File:   JSONQueryParser.cpp
 * Author: Thomas Krause <thomaskrause@posteo.de>
 * 
 * Created on 3. Januar 2016, 16:05
 */

#include "jsonqueryparser.h"
#include "exactannovaluesearch.h"
#include "exactannokeysearch.h"
#include "regexannosearch.h"
#include "operators/precedence.h"
#include <map>

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
  if (alternatives.size() != 0) {
    const auto& firstAlt = alternatives[0];

    // add all nodes
    const auto& nodes = firstAlt["nodes"];

    std::map<std::uint64_t, size_t> nodeIdToPos;
    for (auto it = nodes.begin(); it != nodes.end(); it++) {
      auto& n = *it;
      nodeIdToPos[std::stoull(it.name())] = parseNode(db, n, q);
    }
    
    // add all joins
    const auto& joins = firstAlt["joins"];
    for(auto it = joins.begin(); it != joins.end(); it++) {
      parseJoin(db, *it, q, nodeIdToPos);
    }


  }
  return q;
}

size_t JSONQueryParser::parseNode(const DB& db, const Json::Value node, Query& q) {

  // annotation search?
  if (node["nodeAnnotations"].isArray() && node["nodeAnnotations"].size() > 0) {
    // get the first one
    auto nodeAnno = node["nodeAnnotations"][0];

    return addNodeAnnotation(db, q, optStr(nodeAnno["namespace"]),
            optStr(nodeAnno["name"]), optStr(nodeAnno["value"]),
            optStr(nodeAnno["textMatching"]));

  }// end if annotation search
  else {
    // check for special non-annotation search constructs
    // token search?
    if (node["spannedText"].isString() 
            || (node["token"].isBool() && node["token"].asBool())) {
      return addNodeAnnotation(db, q, optStr(annis_ns), optStr(annis_tok),
              optStr(node["spannedText"]),
              optStr(node["spanTextMatching"]));
    } // end if token has spanned text
    else {
      // just search for any node
      return addNodeAnnotation(db, q, optStr(annis_ns), optStr(annis_node_name),
              optStr(), optStr());
    }
  } // end if special case

}

size_t JSONQueryParser::addNodeAnnotation(const DB& db,
        Query& q,
        const std::shared_ptr<std::string> ns,
        const std::shared_ptr<std::string> name,
        const std::shared_ptr<std::string> value,
        const std::shared_ptr<std::string> textMatching) {

  if (value) {
    // search for the value
    if (*textMatching == "EXACT_EQUAL") {
      // has namespace?
      if (ns) {
        return q.addNode(std::make_shared<ExactAnnoValueSearch>(db,
                *ns,
                *name,
                *value));
      } else {
        return q.addNode(std::make_shared<ExactAnnoValueSearch>(db,
                *name,
                *value));
      }
    } else if (*textMatching == "REGEXP_EQUAL") {
      // has namespace?
      if (ns) {
        return q.addNode(std::make_shared<RegexAnnoSearch>(db,
                *ns,
                *name,
                *value));
      } else {
        return q.addNode(std::make_shared<RegexAnnoSearch>(db,
                *name,
                *value));
      }
    }

  }// end if has value
  else {
    // only search for key
    // has namespace?
    if (ns) {
      return q.addNode(std::make_shared<ExactAnnoKeySearch>(db,
              *ns,
              *name));
    } else {
      return q.addNode(std::make_shared<ExactAnnoKeySearch>(db,
              *name));
    }
  }
}

void JSONQueryParser::parseJoin(const DB& db, const Json::Value join, Query& q, 
        const std::map<std::uint64_t, size_t>& nodeIdToPos ) {
  // get left and right index
  if(join["left"].isUInt64() && join["right"].isUInt64()) {
    auto leftID = join["left"].asUInt64();
    auto rightID = join["right"].asUInt64();
    
    auto itLeft = nodeIdToPos.find(leftID);
    auto itRight = nodeIdToPos.find(rightID);
    
    if(itLeft != nodeIdToPos.end() && itRight != nodeIdToPos.end()) {
      
      auto op = join["op"].asString();
      if(op == "Precedence") {
        q.addOperator(std::make_shared<Precedence>(db,
         join["minDistance"].asUInt(), join["maxDistance"].asUInt()),
                itLeft->second, itRight->second, false);
      }
      
    }
    
  }
}


JSONQueryParser::~JSONQueryParser() {
}

