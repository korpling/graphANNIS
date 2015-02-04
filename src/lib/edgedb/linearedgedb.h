#ifndef LINEAREDGEDB_H
#define LINEAREDGEDB_H

#include "../edgedb.h"
#include "../edgeannotationstorage.h"

#include "../dfs.h"
#include "../exactannokeysearch.h"

#include <fstream>
#include <set>
#include <limits>

#include <boost/archive/binary_oarchive.hpp>
#include <boost/archive/binary_iarchive.hpp>
#include <boost/serialization/map.hpp>
#include <boost/serialization/string.hpp>
#include <boost/serialization/vector.hpp>

#include <boost/format.hpp>

#include <stx/btree_map>


namespace annis
{


template<typename pos_t>
class LinearEdgeDB : public ReadableGraphStorage
{
public:
  class LinearIterator : public EdgeIterator
  {
  public:

    LinearIterator(const LinearEdgeDB& edb, nodeid_t startNode, unsigned int minDistance, unsigned int maxDistance)
      : edb(edb), minDistance(minDistance), maxDistance(maxDistance), startNode(startNode),
        chain(nullptr)
    {
      reset();
    }

    virtual std::pair<bool, nodeid_t> next()
    {
      bool found = false;
      nodeid_t node = 0;
      if(chain != nullptr && currentPos <= endPos && currentPos < chain->size())
      {
        found = true;
        node = chain->at(currentPos);
        chain->at(currentPos);
        currentPos++;
      }
      return std::pair<bool, nodeid_t>(found, node);
    }

    virtual void reset()
    {
      typedef typename stx::btree_map<nodeid_t, RelativePosition<pos_t> >::const_iterator PosIt;
      typedef std::map<nodeid_t, std::vector<nodeid_t> >::const_iterator NodeChainIt;

      PosIt posSourceIt = edb.node2pos.find(startNode);
      if(posSourceIt != edb.node2pos.end())
      {
        const RelativePosition<pos_t>& relPos = posSourceIt->second;
        currentPos = relPos.pos;
        NodeChainIt itNodeChain = edb.nodeChains.find(relPos.root);
        if(itNodeChain != edb.nodeChains.end())
        {
          chain = &(itNodeChain->second);
        }

        // define where to stop
        if(maxDistance == uintmax)
        {
          endPos = std::numeric_limits<pos_t>::max();
        }
        else
        {
          endPos = currentPos + maxDistance;
        }
        // add the minium distance
        currentPos = currentPos + minDistance;

      }
    }

    virtual ~LinearIterator()
    {

    }

  private:

    const LinearEdgeDB& edb;
    unsigned int minDistance;
    unsigned int maxDistance;
    nodeid_t startNode;

    const std::vector<nodeid_t>* chain;
    unsigned int currentPos;
    unsigned int endPos;

  };


public:
  LinearEdgeDB(StringStorage& strings, const Component& component)
    : component(component)
  {

  }

  virtual void clear()
  {
    edgeAnno.clear();
    node2pos.clear();
    nodeChains.clear();
  }

  virtual void copy(const DB& db, const ReadableGraphStorage& orig)
  {
    // find all root nodes
    std::set<nodeid_t> roots;

    // add all nodes to root list
    ExactAnnoKeySearch nodes(db, annis_ns, annis_node_name);

    while(nodes.hasNext())
    {
      nodeid_t n = nodes.next().node;
      // insert all nodes to the root candidate list which are part of this component
      if(!orig.getOutgoingEdges(n).empty())
      {
        roots.insert(n);
      }
    }

    nodes.reset();
    while(nodes.hasNext())
    {
      nodeid_t source = nodes.next().node;

      std::vector<nodeid_t> outEdges = orig.getOutgoingEdges(source);
      for(auto target : outEdges)
      {
        Edge e = {source, target};

        // remove the nodes that have an incoming edge from the root list
        roots.erase(target);

        std::vector<Annotation> edgeAnnos = orig.getEdgeAnnotations(e);
        for(auto a : edgeAnnos)
        {
          edgeAnno.addEdgeAnnotation(e, a);
        }
      }
    }


    for(auto& rootNode : roots)
    {
      // iterate over all edges beginning from the root
      nodeChains[rootNode] = std::vector<nodeid_t>();
      std::vector<nodeid_t>& chain = nodeChains[rootNode];
      chain.push_back(rootNode);
      node2pos[rootNode] = {rootNode, (pos_t) (chain.size()-1)};

      CycleSafeDFS it(orig, rootNode, 1, uintmax);

      uint32_t pos=1;
      for(std::pair<bool, nodeid_t> node = it.next(); node.first; node = it.next(), pos++)
      {
        chain.push_back(node.second);
        node2pos[node.second] = {rootNode, (pos_t)(chain.size()-1)};
      }
    }

    stat = orig.getStatistics();
  }

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const
  {
    typedef typename stx::btree_map<nodeid_t, RelativePosition<pos_t> >::const_iterator PosIt;

    PosIt posSourceIt = node2pos.find(edge.source);
    PosIt posTargetIt = node2pos.find(edge.target);
    if(posSourceIt != node2pos.end() && posTargetIt != node2pos.end())
    {
      RelativePosition<pos_t> posSource = posSourceIt->second;
      RelativePosition<pos_t> posTarget = posTargetIt->second;
      if(posSource.root == posTarget.root && posSource.pos <= posTarget.pos)
      {
        unsigned int diff = posTarget.pos > posSource.pos ?
              posTarget.pos - posSource.pos
            : posSource.pos - posTarget.pos;

        if(diff >= minDistance && diff <= maxDistance)
        {
          return true;
        }
      }
    }
    return false;
  }
  virtual std::unique_ptr<EdgeIterator> findConnected(
                                           nodeid_t sourceNode,
                                           unsigned int minDistance,
                                           unsigned int maxDistance) const
  {
    return std::unique_ptr<EdgeIterator>(new LinearIterator(*this, sourceNode, minDistance, maxDistance));
  }

  virtual int distance(const Edge &edge) const
  {
    typedef typename stx::btree_map<nodeid_t, RelativePosition<pos_t> >::const_iterator PosIt;

    PosIt posSourceIt = node2pos.find(edge.source);
    PosIt posTargetIt = node2pos.find(edge.target);
    if(posSourceIt != node2pos.end() && posTargetIt != node2pos.end())
    {
      RelativePosition<pos_t> posSource = posSourceIt->second;
      RelativePosition<pos_t> posTarget = posTargetIt->second;
      if(posSource.root == posTarget.root && posSource.pos <= posTarget.pos)
      {
        int diff = posTarget.pos - posSource.pos;
        if(diff >= 0)
        {
          return diff;
        }
      }
    }
    return -1;
  }

  virtual bool load(std::string dirPath)
  {
    bool result = ReadableGraphStorage::load(dirPath);

    result = result && edgeAnno.save(dirPath);
    std::ifstream in;


    in.open(dirPath + "/node2pos.btree");
    result = result && node2pos.restore(in);
    in.close();

    in.open(dirPath + "/nodeChains.archive", std::ios::binary);
    boost::archive::binary_iarchive ia(in);
    ia >> nodeChains;
    in.close();

    return result;
  }

  virtual bool save(std::string dirPath)
  {
    bool result = ReadableGraphStorage::save(dirPath);

    result = result && edgeAnno.save(dirPath);

    std::ofstream out;

    out.open(dirPath + "/node2pos.btree");
    node2pos.dump(out);
    out.close();

    out.open(dirPath + "/nodeChains.archive", std::ios::binary);
    boost::archive::binary_oarchive oa(out);
    oa << nodeChains;
    out.close();

    return result;
  }

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const
  {
    return edgeAnno.getEdgeAnnotations(edge);
  }
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const
  {
    std::vector<nodeid_t> result;
    auto it = node2pos.find(node);
    if(it != node2pos.end())
    {
      auto pos = it->second;
      auto chainIt = nodeChains.find(pos.root);
      if(chainIt != nodeChains.end())
      {
        const std::vector<nodeid_t>& chain = chainIt->second;
        if(pos.pos < (chain.size()-1))
        {
          result.push_back(chain[pos.pos+1]);
        }
      }
    }
    return result;
  }
  virtual std::vector<nodeid_t> getIncomingEdges(nodeid_t node) const
  {
    std::vector<nodeid_t> result;
    auto it = node2pos.find(node);
    if(it != node2pos.end())
    {
      auto pos = it->second;
      auto chainIt = nodeChains.find(pos.root);
      if(chainIt != nodeChains.end())
      {
        const std::vector<nodeid_t>& chain = chainIt->second;
        if(pos.pos - 1 > 0 && !chain.empty())
        {
          result.push_back(chain[pos.pos-1]);
        }
      }
    }
    return result;
  }

  virtual std::uint32_t numberOfEdges() const
  {
    return node2pos.size();
  }
  virtual std::uint32_t numberOfEdgeAnnotations() const
  {
    return edgeAnno.numberOfEdgeAnnotations();
  }

  virtual ~LinearEdgeDB()
  {

  }

private:
  const Component& component;
  stx::btree_map<nodeid_t, RelativePosition<pos_t>> node2pos;
  std::map<nodeid_t, std::vector<nodeid_t> > nodeChains;

  EdgeAnnotationStorage edgeAnno;
};

} // end namespace annis

#endif // LINEAREDGEDB_H
