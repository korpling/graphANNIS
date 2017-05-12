#pragma once

#include <annis/operators/operator.h>

#include <annis/db.h>

namespace annis {
class IdenticalNode : public Operator
{
public:
  IdenticalNode(const DB &db);
  virtual ~IdenticalNode();

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs) override;
  virtual bool filter(const Match& lhs, const Match& rhs) override;

  virtual std::string description() override {return "_ident_";}

  virtual bool isCommutative() override {return true;}
private:
  const Annotation anyNodeAnno;
};
}


