#include <annis/operators/inclusion.h>

#include <annis/wrapper.h>

using namespace annis;

Inclusion::Inclusion(const DB &db)
  : db(db),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    tokHelper(db)
{
  gsOrder = db.getGraphStorage(ComponentType::ORDERING, annis_ns, "");
  gsLeftToken = db.getGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "");
  gsRightToken = db.getGraphStorage(ComponentType::RIGHT_TOKEN, annis_ns, "");
  gsCoverage = db.getGraphStorage(ComponentType::COVERAGE, annis_ns, "");

}

bool Inclusion::filter(const Match &lhs, const Match &rhs)
{
  auto lhsTokenRange = tokHelper.leftRightTokenForNode(lhs.node);
  int spanLength = spanLength = gsOrder->distance({lhsTokenRange.first, lhsTokenRange.second});

  auto rhsTokenRange = tokHelper.leftRightTokenForNode(rhs.node);

  if(gsOrder->isConnected({lhsTokenRange.first, rhsTokenRange.first}, 0, spanLength)
     && gsOrder->isConnected({rhsTokenRange.second, lhsTokenRange.second}, 0, spanLength)
    )
  {
    return true;
  }
  return false;
}


std::unique_ptr<AnnoIt> Inclusion::retrieveMatches(const annis::Match &lhs)
{

  std::unique_ptr<ListWrapper> w = std::make_unique<ListWrapper>();

  nodeid_t leftToken;
  nodeid_t rightToken;
  int spanLength = 0;
  if(db.nodeAnnos.getNodeAnnotation(lhs.node, annis_ns, annis_tok).first)
  {
    // is token
    leftToken = lhs.node;
    rightToken = lhs.node;
  }
  else
  {
    leftToken = gsLeftToken->getOutgoingEdges(lhs.node)[0];
    rightToken = gsRightToken->getOutgoingEdges(lhs.node)[0];
    spanLength = gsOrder->distance({leftToken, rightToken});
  }

  // find each token which is between the left and right border
  std::unique_ptr<EdgeIterator> itIncludedStart = gsOrder->findConnected(leftToken, 0, spanLength);
  for(std::pair<bool, nodeid_t> includedStart = itIncludedStart->next();
      includedStart.first;
      includedStart = itIncludedStart->next())
  {
    const nodeid_t& includedTok = includedStart.second;
    // add the token itself
    w->addMatch({includedTok, anyNodeAnno});

    // add aligned nodes
    for(const auto& leftAlignedNode : gsLeftToken->getOutgoingEdges(includedTok))
    {
      nodeid_t includedEndCandiate = gsRightToken->getOutgoingEdges(leftAlignedNode)[0];
      if(gsOrder->isConnected({includedEndCandiate, rightToken}, 0, spanLength))
      {
        w->addMatch({leftAlignedNode, anyNodeAnno});
      }
    }
  }

  return w;
}

double Inclusion::selectivity() 
{
  if(gsOrder == nullptr || gsCoverage == nullptr)
  {
    return Operator::selectivity();
  }
  auto statsCov = gsCoverage->getStatistics();
  auto statsOrder = gsOrder->getStatistics();
  if(statsCov.nodes == 0)
  {
    // only token in this corpus
    return 1.0 / (double) statsOrder.nodes;
  }
  else
  {
    // The fan-out is the selectivity for the number of covered token.
    // Use a constant that dependends on the number of token to estimate the number of included
    // nodes.
    // TODO: which statistics do we need to calculate the better number?
    return statsCov.avgFanOut * 0.5; 
  }
}



Inclusion::~Inclusion()
{
}


