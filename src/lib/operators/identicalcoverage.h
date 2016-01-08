/* 
 * File:   IdenticalCoverage.h
 * Author: thomas
 *
 * Created on 8. Januar 2016, 13:58
 */

#ifndef IDENTICALCOVERAGE_H
#define IDENTICALCOVERAGE_H

#include "../operator.h"
#include <db.h>
#include "helper.h"

namespace annis
{

class IdenticalCoverage : public Operator
{
public:
  IdenticalCoverage(const DB &db);
  IdenticalCoverage(const IdenticalCoverage& orig) = delete;
  
  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);
  virtual bool isReflexive() {return true;};
  
  virtual ~IdenticalCoverage();
private:
  
  const DB &db;
  TokenHelper tokHelper;
  const ReadableGraphStorage* gsOrder;
  const ReadableGraphStorage* gsLeftToken;
  const ReadableGraphStorage* gsRightToken;
  
  Annotation anyNodeAnno;

};

} // end namespace annis

#endif /* IDENTICALCOVERAGE_H */

