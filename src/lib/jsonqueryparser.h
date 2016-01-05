/* 
 * File:   JSONQueryParser.h
 * Author: Thomas Krause <thomaskrause@posteo.de>
 *
 * Created on 3. Januar 2016, 16:05
 */

#ifndef JSONQUERYPARSER_H
#define JSONQUERYPARSER_H

#include "query.h"
#include <json/json.h>
#include <boost/optional.hpp>

namespace annis {

  class JSONQueryParser {
  public:
    JSONQueryParser();
    JSONQueryParser(const JSONQueryParser& orig) = delete;
    JSONQueryParser &operator=(const JSONQueryParser&) = delete;

    static std::shared_ptr<Query> parse(const DB& db, std::istream& json,
      bool useNestedLoop=false);

    virtual ~JSONQueryParser();
  private:
    
    static size_t parseNode(const DB& db, const Json::Value node, std::shared_ptr<Query>);
    static size_t addNodeAnnotation(const DB& db,
        std::shared_ptr<Query> q,
        boost::optional<std::string> ns,
        boost::optional<std::string> name, 
        boost::optional<std::string> value,
        boost::optional<std::string> textMatching);
    
    static void parseJoin(const DB& db, const Json::Value join, 
      std::shared_ptr<Query> q, const  std::map<std::uint64_t, size_t>& nodeIdToPos,
      bool useNestedLoop=false);
    
    static boost::optional<std::string> optStr(const Json::Value& val) {
      if(val.isString()) {
        return boost::optional<std::string>(val.asString());
      } else {
        return boost::optional<std::string>();
      }
    }
    
    static boost::optional<std::string> optStr(const std::string& val) {
      return boost::optional<std::string>(val);
    }
    
    static boost::optional<std::string> optStr() {
      return boost::optional<std::string>();
    }
    
    static Annotation getEdgeAnno(const DB& db, const Json::Value& edgeAnno);

  };

}

#endif /* JSONQUERYPARSER_H */

