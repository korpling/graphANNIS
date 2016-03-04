#pragma once

#include <set>
#include <list>

#include <annis/db.h>
#include <annis/operators/operator.h>
#include <annis/util/helper.h>

namespace annis
{

class Inclusion : public Operator
{
public:
  Inclusion(const DB &db);

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);

  virtual bool isReflexive() override {return false;}
  
  virtual std::string description() override
  {
    return "_i_";
  }

  
  virtual double selectivity() override;


  virtual ~Inclusion();
private:

  const DB& db;
  const ReadableGraphStorage* gsOrder;
  const ReadableGraphStorage* gsLeftToken;
  const ReadableGraphStorage* gsRightToken;
  const ReadableGraphStorage* gsCoverage;

  Annotation anyNodeAnno;

  TokenHelper tokHelper;


};
} // end namespace annis
