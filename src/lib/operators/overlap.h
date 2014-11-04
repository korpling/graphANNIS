#ifndef OVERLAP_H
#define OVERLAP_H

#include <set>

#include "../db.h"
#include "../annotationiterator.h"

namespace annis
{

class Overlap : public BinaryOperatorIterator
{
public:
  Overlap(DB &db, AnnotationIterator& left, AnnotationIterator& right);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~Overlap();
private:

  AnnotationIterator& left;
  Annotation rightAnnotation;

  const DB& db;
  const EdgeDB* edbLeft;
  const EdgeDB* edbRight;

  nodeid_t leftTokBorder;
  nodeid_t rightTokBorder;
  nodeid_t currentTok;

  std::set<nodeid_t> nodesOverlappingCurrentToken;
  std::set<nodeid_t>::const_iterator itNodeOverlappingCurrentToken;

  std::vector<Annotation> currentAnnnotations;
  std::vector<Annotation>::const_iterator itCurrentAnnotations;

  Match currentLeftMatch;
  Match currentRightMatch;

  std::set<BinaryMatch, compBinaryMatch> uniqueMatches;

  bool nextAnnotation();
  bool nextOverlappingNode();
  bool nextToken();
  bool nextMatch();

};
} // end namespace annis
#endif // OVERLAP_H
