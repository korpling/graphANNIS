#ifndef OVERLAP_H
#define OVERLAP_H

#include <set>
#include <list>

#include "../db.h"
#include "../annotationiterator.h"
#include "defaultjoins.h"
#include "../helper.h"
#include "operator.h"

namespace annis
{

class Overlap : public Operator
{
public:

  Overlap(DB &db);

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);

  virtual ~Overlap();
private:
  DB& db;
  TokenHelper tokHelper;
  Annotation anyNodeAnno;
  const EdgeDB* edbOrder;
  const EdgeDB* edbCoverage;
};
} // end namespace annis
#endif // OVERLAP_H
