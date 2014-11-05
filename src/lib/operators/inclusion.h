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
  std::set<BinaryMatch, compBinaryMatch> uniqueMatches;

  // the following variales hold the current iteration state
  std::vector<Annotation> currentAnnnotations;
  std::vector<Annotation>::const_iterator itCurrentAnnotations;

  Match currentRightMatch;

  std::vector<nodeid_t> rightMatchCandidates;
  std::vector<nodeid_t>::const_iterator itRightMatchCandidates;

  EdgeIterator* itCurrentCoveredToken;

  Match currentLeftMatch;
  // end iteration sttate


  bool nextAnnotation();
  bool nextRightMatch();
  bool nextCoveredToken();
  bool nextLeftMatch();

};
} // end namespace annis
#endif // INCLUSION_H
