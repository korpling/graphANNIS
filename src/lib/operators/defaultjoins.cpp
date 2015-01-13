#include "defaultjoins.h"
#include "annotationsearch.h"

using namespace annis;

RightMostTokenForNodeIterator::RightMostTokenForNodeIterator(std::shared_ptr<AnnoIt> source, const DB &db)
  : source(source), db(db), edb(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "")), tokenShortcut(false)
{
  anyTokAnnotation = Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
  const Annotation& anno = source->getAnnotation();
  if(anno.name == db.getTokStringID() && anno.ns == db.getNamespaceStringID() && anno.val == 0)
  {
    tokenShortcut = true;
  }
}

bool RightMostTokenForNodeIterator::hasNext()
{
  return source->hasNext();
}

Match RightMostTokenForNodeIterator::next()
{
  Match result;
  if(source->hasNext() && edb != NULL)
  {
    currentOriginalMatch = source->next();

    // check if we can use the shortcut
    if(tokenShortcut)
    {
      return currentOriginalMatch;
    }

    // we still have to check if this is a token in case for token annotions
    for(const auto& a : db.getNodeAnnotationsByID(currentOriginalMatch.node))
    {
      if(checkAnnotationEqual(anyTokAnnotation, a))
      {
        return currentOriginalMatch;
      }
    }

    result.node = edb->getOutgoingEdges(currentOriginalMatch.node)[0];
    result.anno.name = db.getTokStringID();
    result.anno.ns = db.getNamespaceStringID();
    result.anno.val = 0; //TODO: do we want to include the actual value here?
  }

  return result;
}

void RightMostTokenForNodeIterator::reset()
{
  source->reset();
}

const Match& RightMostTokenForNodeIterator::currentNodeMatch()
{
  return currentOriginalMatch;
}


LeftMostTokenForNodeIterator::LeftMostTokenForNodeIterator(AnnoIt &source, const DB &db)
  : source(source), db(db), edb(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, ""))
{
  anyTokAnnotation = Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
}

bool LeftMostTokenForNodeIterator::hasNext()
{
  return source.hasNext();
}

Match LeftMostTokenForNodeIterator::next()
{
  Match result;
  if(source.hasNext() && edb != NULL)
  {
    currentOriginalMatch = source.next();

    // check if this is a token
    for(const auto& a : db.getNodeAnnotationsByID(currentOriginalMatch.node))
    {
      if(checkAnnotationEqual(anyTokAnnotation, a))
      {
        return currentOriginalMatch;
      }
    }

    result.node = edb->getOutgoingEdges(currentOriginalMatch.node)[0];
    result.anno.name = db.getTokStringID();
    result.anno.ns = db.getNamespaceStringID();
    result.anno.val = 0; //TODO: do we want to include the actual value here?
  }

  return result;
}

void LeftMostTokenForNodeIterator::reset()
{
  source.reset();
}

Match LeftMostTokenForNodeIterator::currentNodeMatch()
{
  return currentOriginalMatch;
}




NestedLoopJoin::NestedLoopJoin(std::shared_ptr<Operator> op)
  : op(op), initialized(false)
{
}

BinaryMatch NestedLoopJoin::next()
{
  BinaryMatch result;
  result.found = false;

  if(!op || !left || !right)
  {
    return result;
  }

  bool proceed = true;

  if(!initialized)
  {
    proceed = false;
    if(left->hasNext())
    {
      matchLeft = left->next();
      proceed = true;
      initialized = true;
    }
  }

  while(proceed)
  {

    while(right->hasNext())
    {
      matchRight = right->next();

      if(op->filter(matchLeft, matchRight))
      {
        result.found = true;
        result.lhs = matchLeft;
        result.rhs = matchRight;

        return result;
      }
    }
    if(left->hasNext())
    {
      matchLeft = left->next();
      right->reset();
    }
    else
    {
      proceed = false;
    }
  }
  return result;
}

void NestedLoopJoin::reset()
{
  left->reset();
  right->reset();
  initialized = false;
}


void NestedLoopJoin::init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
{
  left = lhs;
  right = rhs;
  initialized = false;
}

NestedLoopJoin::~NestedLoopJoin()
{

}

SeedJoin::SeedJoin(const DB &db, std::shared_ptr<Operator> op)
  : db(db), op(op), currentMatchValid(false), anyNodeShortcut(false)
{
  reset();
}

BinaryMatch SeedJoin::next()
{
  currentMatch.found = false;

  if(!op || !left || !currentMatchValid)
  {
    return currentMatch;
  }

  if(nextRightAnnotation())
  {
    return currentMatch;
  }

  do
  {
    while(matchesByOperator && matchesByOperator->hasNext())
    {
      currentMatch.rhs = matchesByOperator->next();

      if(anyNodeShortcut)
      {
        currentMatch.found = true;
        std::pair<bool, Annotation> annoSearch =
            db.getNodeAnnotation(currentMatch.rhs.node, db.getNamespaceStringID(),
                                 db.getNodeNameStringID());
        if(annoSearch.first)
        {
          currentMatch.rhs.anno = annoSearch.second;
        }
        return currentMatch;
      }
      else
      {
        // check all annotations which of them matches
        std::list<Annotation> annos = db.getNodeAnnotationsByID(currentMatch.rhs.node);
        for(const auto& a : annos)
        {
          if(checkAnnotationEqual(a, right))
          {
            matchingRightAnnos.push_back(a);
          }
        }
        if(nextRightAnnotation())
        {
          return currentMatch;
        }
      }
    } // end while there are right candidates
  } while(nextLeftMatch()); // end while left has match


  return currentMatch;
}


void SeedJoin::init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
{
  left = lhs;
  anyNodeShortcut = false;
  if(rhs)
  {
    Annotation anyNodeAnno = Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID());
    if(checkAnnotationEqual(rhs->getAnnotation(), anyNodeAnno))
    {
      anyNodeShortcut = true;
    }
    else
    {
      right = rhs->getAnnotation();
    }
  }
  else
  {
    anyNodeShortcut = true;
  }

  nextLeftMatch();
}


void SeedJoin::reset()
{
  if(left)
  {
    left->reset();
  }

  currentMatch.lhs.node = 6666666;

  matchesByOperator.release();
  matchingRightAnnos.clear();
  currentMatchValid = false;
  anyNodeShortcut = false;

  // start the iterations
  nextLeftMatch();

}

bool SeedJoin::nextLeftMatch()
{
  if(left && left->hasNext())
  {
    matchesByOperator.release();
    matchingRightAnnos.clear();

    currentMatch.lhs = left->next();
    currentMatchValid = true;

    matchesByOperator = op->retrieveMatches(currentMatch.lhs);
    if(!matchesByOperator)
    {
      std::cerr << "could not create right matches from operator!" << std::endl;
    }


    return true;
  }

  return false;
}

bool SeedJoin::nextRightAnnotation()
{
  if(matchingRightAnnos.size() > 0)
  {
    currentMatch.found = true;
    currentMatch.rhs.anno = matchingRightAnnos.front();
    matchingRightAnnos.pop_front();
    return true;
  }
  return false;
}

SeedJoin::~SeedJoin()
{
}

