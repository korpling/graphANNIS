#ifndef DEFAULTJOINS_H
#define DEFAULTJOINS_H

#include "types.h"
#include "annotationiterator.h"
#include "operator.h"
#include "edgedb.h"
#include "db.h"

namespace annis
{

/** A join that checks all combinations of the left and right matches if their are connected. */
class LegacyNestedLoopJoin : public BinaryIt
{
public:
  LegacyNestedLoopJoin(const EdgeDB* edb, std::shared_ptr<AnnoIt> left, std::shared_ptr<AnnoIt> right,
                 unsigned int minDistance = 1, unsigned int maxDistance = 1);
  virtual ~LegacyNestedLoopJoin();

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();
private:
  const EdgeDB* edb;
  std::shared_ptr<AnnoIt> left;
  std::shared_ptr<AnnoIt> right;
  unsigned int minDistance;
  unsigned int maxDistance;
  bool initialized;

  Match matchLeft;
  Match matchRight;

};

/** A join that takes the left argument as a seed, finds all connected nodes (matching the distance) and checks the condition for each node. */
class LegacySeedJoin : public BinaryIt
{
public:
  LegacySeedJoin(const DB& db, const EdgeDB* edb, std::shared_ptr<AnnoIt> left, Annotation right,
                 unsigned int minDistance = 1, unsigned int maxDistance = 1);
  virtual ~LegacySeedJoin();

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();
private:
  const DB& db;
  const EdgeDB* edb;
  std::shared_ptr<AnnoIt> left;
  Annotation right;
  unsigned int minDistance;
  unsigned int maxDistance;

  Match matchLeft;

  EdgeIterator* edgeIterator;
  std::pair<bool, nodeid_t> connectedNode;
  std::list<Annotation> candidateAnnotations;
  std::list<Annotation>::const_iterator currentAnnotationCandidate;
  bool anyNodeShortcut;

  bool nextLeft();
  bool nextConnected();
  bool nextAnnotation();

};


/**
 * @brief The RightMostTokenForNodeIterator class
 *
 * This iterator outputs the token that is right aligned with the original matched node.
 * If the matched node itself is a token, the token is returned.
 */
class RightMostTokenForNodeIterator : public AnnoIt
{
public:

  RightMostTokenForNodeIterator(std::shared_ptr<AnnoIt> source, const DB& db);

  virtual bool hasNext();
  virtual Match next();
  virtual void reset();

  virtual const Match &currentNodeMatch();

  virtual const Annotation& getAnnotation() {return source->getAnnotation();}

  virtual ~RightMostTokenForNodeIterator() {}


private:
  std::shared_ptr<AnnoIt> source;
  const DB& db;
  const EdgeDB* edb;
  Match matchTemplate;
  Match currentOriginalMatch;
  Annotation anyTokAnnotation;
  bool tokenShortcut;

  void initEdgeDB();
};

/**
 * @brief The LeftMostTokenForNodeIterator class
 *
 * This iterator outputs the token that is left aligned with the original matched node.
 * If the matched node itself is a token, the token is returned.
 */
class LeftMostTokenForNodeIterator : public AnnoIt
{
public:

  LeftMostTokenForNodeIterator(AnnoIt& source, const DB& db);

  virtual bool hasNext();
  virtual Match next();
  virtual void reset();

  virtual Match currentNodeMatch();

  virtual const Annotation& getAnnotation() {return source.getAnnotation();}

  virtual ~LeftMostTokenForNodeIterator() {}


private:
  AnnoIt& source;
  const DB& db;
  const EdgeDB* edb;
  Match matchTemplate;
  Match currentOriginalMatch;
  Annotation anyTokAnnotation;

  void initEdgeDB();
};

/** A join that checks all combinations of the left and right matches if their are connected. */
class NestedLoopJoin : public Join
{
public:
  NestedLoopJoin(std::shared_ptr<Operator> op);
  virtual ~NestedLoopJoin();

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();
private:
  std::shared_ptr<Operator> op;
  bool initialized;

  std::shared_ptr<AnnoIt> left;
  std::shared_ptr<AnnoIt> right;

  Match matchLeft;
  Match matchRight;

};


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

#endif // DEFAULTJOINS_H
