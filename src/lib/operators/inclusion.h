#ifndef INCLUSION_H
#define INCLUSION_H

#include <set>
#include <list>

#include "../db.h"
#include "../annotationiterator.h"

namespace annis
{

class Inclusion : public Join
{
public:
  Inclusion(DB &db);

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~Inclusion();
private:

  std::shared_ptr<AnnoIt> left;
  Annotation rightAnnotation;

  const DB& db;
  std::vector<const EdgeDB*> edbCoverage;
  const EdgeDB* edbOrder;
  const EdgeDB* edbLeftToken;
  const EdgeDB* edbRightToken;

  // the following variales hold the current iteration state
  std::list<Match> currentMatches;
  BinaryMatch currentResult;
  // end iteration state


};
} // end namespace annis
#endif // INCLUSION_H
