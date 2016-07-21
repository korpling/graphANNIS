#include <annis/edgeannotationstorage.h>

#include <annis/serializers.h>

#include <fstream>

#include <boost/archive/binary_iarchive.hpp>
#include <boost/archive/binary_oarchive.hpp>


using namespace annis;

EdgeAnnotationStorage::EdgeAnnotationStorage()
{

}

EdgeAnnotationStorage::~EdgeAnnotationStorage()
{

}

void EdgeAnnotationStorage::clear()
{
  edgeAnnotations.clear();
}

bool EdgeAnnotationStorage::load(std::string dirPath)
{
  std::ifstream in;
  in.open(dirPath + "/edgeAnnotations.archive");

  if(in.is_open())
  {
    boost::archive::binary_iarchive iaEdgeAnnotations(in);
    iaEdgeAnnotations >> edgeAnnotations;
    in.close();
  }

  return true;
}

bool EdgeAnnotationStorage::save(std::string dirPath)
{
  std::ofstream out;
  out.open(dirPath + "/edgeAnnotations.archive");
  boost::archive::binary_oarchive oaEdgeAnnotations(out);
  oaEdgeAnnotations << edgeAnnotations;
  out.close();

  return true;
}

