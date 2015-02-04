#include "coverageedb.h"

#include <fstream>
#include <set>

#include <boost/archive/binary_iarchive.hpp>
#include <boost/archive/binary_oarchive.hpp>

#include <boost/serialization/utility.hpp>
#include <boost/serialization/collections_save_imp.hpp>
#include <boost/serialization/collections_load_imp.hpp>
#include <boost/serialization/split_free.hpp>

using namespace annis;


namespace boost
{
namespace serialization
{
// unordered_multimap
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void save(
    Archive & ar,
    const std::unordered_multimap<Key, Type, Compare, Allocator> &t,
    const unsigned int /* file_version */
    ){
  boost::serialization::stl::save_collection<
      Archive,
      std::unordered_multimap<Key, Type, Compare, Allocator>
      >(ar, t);
}

template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void load(
    Archive & ar,
    std::unordered_multimap<Key, Type, Compare, Allocator> &t,
    const unsigned int /* file_version */
    ){
  boost::serialization::stl::load_collection<
      Archive,
      std::unordered_multimap<Key, Type, Compare, Allocator>,
      boost::serialization::stl::archive_input_map<
      Archive, std::unordered_multimap<Key, Type, Compare, Allocator>
      >,
      boost::serialization::stl::no_reserve_imp<
      std::unordered_multimap<Key, Type, Compare, Allocator>
      >
      >(ar, t);
}

// split non-intrusive serialization function member into separate
// non intrusive save/load member functions
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void serialize(
    Archive & ar,
    std::unordered_multimap<Key, Type, Compare, Allocator> &t,
    const unsigned int file_version
    ){
  boost::serialization::split_free(ar, t, file_version);
}
} // serialization
} // end namespace boost

CoverageEdgeDB::CoverageEdgeDB(StringStorage &strings, const Component &component)
  : FallbackEdgeDB(strings, component)
{
}

void CoverageEdgeDB::calculateIndex()
{
  typedef stx::btree_set<Edge>::const_iterator EdgeIt;
  for(EdgeIt it=getEdgesBegin();
      it != getEdgesEnd(); it++)

  {
    const Edge& e = *it;
    coveringNodes.insert(std::pair<nodeid_t, nodeid_t>(e.target, e.source));
  }
}

bool CoverageEdgeDB::save(std::string dirPath)
{
  bool result = FallbackEdgeDB::save(dirPath);

  std::ofstream out;

  out.open(dirPath + "/coveringNodes.archive");
  boost::archive::binary_oarchive oa(out);
  oa << coveringNodes;
  out.close();


  return result;
}

bool CoverageEdgeDB::load(std::string dirPath)
{
  bool result = FallbackEdgeDB::load(dirPath);
  std::ifstream in;

  in.open(dirPath + "/coveringNodes.archive");
  boost::archive::binary_iarchive ia(in);
  ia >> coveringNodes;
  in.close();

//  for(stx::btree_multimap<nodeid_t, nodeid_t>::const_iterator it=coveringNodes.begin();
//      it != coveringNodes.end(); it++)
//  {
//    std::cout << "covering: " <<  it->first << "->" << it->second << std::endl;
//  }

  return result;
}

std::vector<nodeid_t> CoverageEdgeDB::getIncomingEdges(nodeid_t node) const
{
  typedef std::unordered_multimap<nodeid_t, nodeid_t>::const_iterator It;

  std::vector<nodeid_t> result;
  result.reserve(20);

  std::pair<It, It> itRange = coveringNodes.equal_range(node);
  for(It it=itRange.first; it != itRange.second; it++)
  {
    result.push_back(it->second);
  }

  return result;
}

int CoverageEdgeDB::distance(const Edge &edge) const
{
  // coverage components only have paths of length 1
  if(FallbackEdgeDB::isConnected(edge, 1, 1))
  {
    return 1;
  }

  // not connected at all
  return -1;
}

bool CoverageEdgeDB::isConnected(const Edge &edge, unsigned int /*minDistance*/, unsigned int /*maxDistance*/) const
{
  // coverage components only have paths of length 1
  if(FallbackEdgeDB::isConnected(edge, 1, 1))
  {
    return true;
  }

  // not connected at all
  return false;
}


CoverageEdgeDB::~CoverageEdgeDB()
{

}
