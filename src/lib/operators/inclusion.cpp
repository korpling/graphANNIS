#include "inclusion.h"

#include "../componenttypeiterator.h"

using namespace annis;

Inclusion::Inclusion(DB &db, AnnotationIterator &left, AnnotationIterator &right)
  : left(left), rightAnnotation(right.getAnnotation()), db(db)
{
  edbCoverage = db.getAllEdgeDBForType(ComponentType::COVERAGE);
  edbOrder = db.getEdgeDB(ComponentType::ORDERING, annis_ns, "");
  edbLeftToken = db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "");
  edbRightToken = db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "");
  reset();
}

BinaryMatch Inclusion::next()
{
  BinaryMatch result;
  result.found = false;

  while(currentMatches.empty() && left.hasNext())
  {
    result.lhs = left.next();

    currentMatches.clear();

    nodeid_t leftToken;
    nodeid_t rightToken;
    int spanLength = 0;
    if(db.getNodeAnnotation(result.lhs.node, annis_ns, annis_tok).first)
    {
      // is token
      leftToken = result.lhs.node;
      rightToken = result.lhs.node;
    }
    else
    {
      leftToken = edbLeftToken->getOutgoingEdges(result.lhs.node)[0];
      rightToken = edbRightToken->getOutgoingEdges(result.lhs.node)[0];
      spanLength = edbOrder->distance(initEdge(leftToken, rightToken));
    }

    // find each token which is between the left and right border
    EdgeIterator* itIncludedStart = edbOrder->findConnected(leftToken, 0, spanLength);
    for(std::pair<bool, nodeid_t> includedStart = itIncludedStart->next();
        includedStart.first;
        includedStart = itIncludedStart->next())
    {
      for(Annotation anno : db.getNodeAnnotationsByID(includedStart.second))
      {
        if(checkAnnotationEqual(rightAnnotation, anno))
        {
          Match m;
          m.anno = anno;
          m.node = includedStart.second;
          currentMatches.push_back(m);
          break;
        }
      }
      for(auto leftAlignedNode : edbLeftToken->getOutgoingEdges(includedStart.second))
      {
        nodeid_t includedEndCandiate = edbRightToken->getOutgoingEdges(leftAlignedNode)[0];
        if(edbOrder->isConnected(initEdge(includedEndCandiate, rightToken), 0, std::numeric_limits<unsigned int>::max()))
        {

          for(Annotation anno : db.getNodeAnnotationsByID(leftAlignedNode))
          {
            if(checkAnnotationEqual(rightAnnotation, anno))
            {
              Match m;
              m.anno = anno;
              m.node = leftAlignedNode;
              currentMatches.push_back(m);

              break;
            }
          }

        }
      }
    }
  }

  if(!currentMatches.empty())
  {
    result.found = true;
    result.rhs = currentMatches.front();
    currentMatches.pop_front();
  }

  return result;
}

void Inclusion::reset()
{
  uniqueMatches.clear();
  left.reset();
}

Inclusion::~Inclusion()
{
}
