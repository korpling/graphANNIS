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

namespace annis {

  class JSONQueryParser {
  public:
    JSONQueryParser();
    JSONQueryParser(const JSONQueryParser& orig) = delete;
    JSONQueryParser &operator=(const JSONQueryParser&) = delete;

    static Query parse(const DB& db, std::istream& json);

    virtual ~JSONQueryParser();
  private:
    
    static size_t parseNode(const DB& db, const Json::Value node, Query& q);
    static size_t addNodeAnnotation(const DB& db,
        Query& q,
        const std::shared_ptr<std::string> ns,
        const std::shared_ptr<std::string> name, 
        const std::shared_ptr<std::string> value,
        const std::shared_ptr<std::string> textMatching);
    
    static void parseJoin(const DB& db, const Json::Value join, Query& q);
    
    static std::shared_ptr<std::string> optStr(const Json::Value& val) {
      if(val.isString()) {
        return std::make_shared<std::string>(val.asString());
      } else {
        return std::shared_ptr<std::string>();
      }
    }
    
    static std::shared_ptr<std::string> optStr(const std::string& val) {
      return std::make_shared<std::string>(val);
    }
    
    static std::shared_ptr<std::string> optStr() {
      return std::shared_ptr<std::string>();
    }

  };

}

#endif /* JSONQUERYPARSER_H */

