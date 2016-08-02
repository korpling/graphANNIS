#pragma once

#include <set>
#include <list>

#include <annis/db.h>
#include <annis/iterators.h>
#include <annis/util/helper.h>
#include <annis/operators/operator.h>

namespace annis
{

class Overlap : public Operator
{
public:

  Overlap(const DB &db, GraphStorageHolder &gsh);

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs) override;
  virtual bool filter(const Match& lhs, const Match& rhs) override;


  virtual bool isReflexive() override {return false;}
  virtual bool isCommutative() override {return true;}

  virtual std::string description() override
  {
    return "_o_";
  }

  virtual double selectivity() override;

  
  virtual ~Overlap();
private:
  const DB& db;
  TokenHelper tokHelper;
  Annotation anyNodeAnno;
  std::shared_ptr<const ReadableGraphStorage> gsOrder;
  std::shared_ptr<const ReadableGraphStorage> gsCoverage;
  std::shared_ptr<const ReadableGraphStorage> gsInverseCoverage;
};
} // end namespace annis
