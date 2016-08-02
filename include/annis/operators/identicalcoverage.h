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
  IdenticalCoverage(const DB &db, GraphStorageHolder &gsh);
  IdenticalCoverage(const IdenticalCoverage& orig) = delete;
  
  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs) override;
  virtual bool filter(const Match& lhs, const Match& rhs) override;
  virtual bool isReflexive() override {return false;}
  virtual bool isCommutative() override {return true;}

  virtual std::string description() override
  {
    return "_=_";
  }

  virtual double selectivity() override;

  
  virtual ~IdenticalCoverage();
private:
  
  const DB &db;
  TokenHelper tokHelper;
  std::shared_ptr<const ReadableGraphStorage> gsOrder;
  std::shared_ptr<const ReadableGraphStorage> gsLeftToken;
  std::shared_ptr<const ReadableGraphStorage> gsRightToken;
  std::shared_ptr<const ReadableGraphStorage> gsCoverage;
  
  Annotation anyNodeAnno;

};

} // end namespace annis


