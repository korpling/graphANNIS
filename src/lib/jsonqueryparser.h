/* 
 * File:   JSONQueryParser.h
 * Author: Thomas Krause <thomaskrause@posteo.de>
 *
 * Created on 3. Januar 2016, 16:05
 */

#ifndef JSONQUERYPARSER_H
#define JSONQUERYPARSER_H

#include "query.h"

namespace annis {

  class JSONQueryParser {
  public:
    JSONQueryParser();
    JSONQueryParser(const JSONQueryParser& orig) = delete;
    JSONQueryParser &operator=(const JSONQueryParser&) = delete;

    static Query parse(const DB& db, std::istream& json);

    virtual ~JSONQueryParser();
  private:

  };

}

#endif /* JSONQUERYPARSER_H */

