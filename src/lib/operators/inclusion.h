#ifndef INCLUSION_H
#define INCLUSION_H

#include <set>
#include <list>

#include "../db.h"
#include "../annotationiterator.h"

namespace annis
{

class Inclusion : public BinaryOperatorIterator
{
public:
  Inclusion(DB &db, AnnotationIterator& left, AnnotationIterator& right);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~Inclusion();
private:

  AnnotationIterator& left;
  Annotation rightAnnotation;

  const DB& db;
  std::vector<const EdgeDB*> edbCoverage;
  const EdgeDB* edbOrder;
  const EdgeDB* edbLeftToken;
  const EdgeDB* edbRightToken;
  std::set<BinaryMatch, compBinaryMatch> uniqueMatches;

  // the following variales hold the current iteration state
  std::list<Match> currentMatches;
  // end iteration state


};
} // end namespace annis
#endif // INCLUSION_H
