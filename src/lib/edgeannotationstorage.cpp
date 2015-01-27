#include "edgeannotationstorage.h"
#include <fstream>

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
  in.open(dirPath + "/edgeAnnotations.btree");
  edgeAnnotations.restore(in);
  in.close();
}

bool EdgeAnnotationStorage::save(std::string dirPath)
{
  std::ofstream out;
  out.open(dirPath + "/edgeAnnotations.btree");
  edgeAnnotations.dump(out);
  out.close();

}

