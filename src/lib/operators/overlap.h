#ifndef OVERLAP_H
#define OVERLAP_H

#include <set>
#include <list>

#include <db.h>
#include <iterators.h>
#include <util/helper.h>
#include "operator.h"

namespace annis
{

class Overlap : public Operator
{
public:

  Overlap(const DB &db);

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);


  virtual bool isReflexive() {return false;}

  virtual ~Overlap();
private:
  const DB& db;
  TokenHelper tokHelper;
  Annotation anyNodeAnno;
  const ReadableGraphStorage* gsOrder;
  const ReadableGraphStorage* gsCoverage;
  const ReadableGraphStorage* gsInverseCoverage;
};
} // end namespace annis
#endif // OVERLAP_H
