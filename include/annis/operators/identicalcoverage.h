/* 
 * File:   IdenticalCoverage.h
 * Author: thomas
 *
 * Created on 8. Januar 2016, 13:58
 */

#pragma once

#include <annis/operators/operator.h>
#include <annis/db.h>
#include <annis/util/helper.h>

namespace annis
{

class IdenticalCoverage : public Operator
{
public:
  IdenticalCoverage(const DB &db);
  IdenticalCoverage(const IdenticalCoverage& orig) = delete;
  
  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);
  virtual bool isReflexive() override {return false;};
  virtual bool isCommutative() override {return true;}

  
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


