#pragma once

#include <annis/db.h>
#include <annis/util/helper.h>
#include <annis/operators/operator.h>

#include <list>
#include <stack>

namespace annis
{

class Precedence : public Operator
{
public:

  Precedence(const DB& db, GraphStorageHolder &gsh, unsigned int minDistance=1, unsigned int maxDistance=1);

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs) override;
  virtual bool filter(const Match& lhs, const Match& rhs) override;
  
  virtual std::string description() override;

  virtual double selectivity() override;

  
  virtual ~Precedence();
private:
  TokenHelper tokHelper;
  std::shared_ptr<const ReadableGraphStorage> gsOrder;
  std::shared_ptr<const ReadableGraphStorage> gsLeft;
  Annotation anyTokAnno;
  Annotation anyNodeAnno;

  unsigned int minDistance;
  unsigned int maxDistance;
};

} // end namespace annis

