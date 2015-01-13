#ifndef SEED_H
#define SEED_H

#include "types.h"
#include "iterators.h"
#include "operator.h"
#include "join.h"
#include "edgedb.h"
#include "db.h"

namespace annis
{

/** A join that takes the left argument as a seed, finds all connected nodes (matching the distance) and checks the condition for each node. */
class SeedJoin : public Join
{
public:
  SeedJoin(const DB& db, std::shared_ptr<Operator> op);
  virtual ~SeedJoin();

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();
private:
  const DB& db;
  std::shared_ptr<Operator> op;

  std::shared_ptr<AnnoIt> left;
  Annotation right;
  unsigned int minDistance;
  unsigned int maxDistance;

  std::unique_ptr<AnnoIt> matchesByOperator;
  BinaryMatch currentMatch;
  bool currentMatchValid;
  std::list<Annotation> matchingRightAnnos;

  bool anyNodeShortcut;

  bool nextLeftMatch();
  bool nextRightAnnotation();

};


} // end namespace annis

#endif // SEED_H
