bench_getaql <- function(x, querydir) {
  aqlFile <- ""
  group <- x[1]
  problemSpace = as.numeric(x[2])
  if(problemSpace == 0) {
    corpus <- sub("_[^_]+$", "", group)
    fn <- substr(group, nchar(corpus)+2, nchar(group))
    
    aqlFile <- paste(querydir, "/", corpus,"/", fn, ".aql", sep='')
  } else {
    aqlFile <- paste(querydir, "/", x[1] ,"/", formatC(problemSpace, width=5, flag="0",  mode="integer"), ".aql", sep='')
  }
  
  aql <- readChar(aqlFile, file.info(aqlFile)$size)
  
  return(aql)
}
bench_extract <- function(fn, querydir) {
  d = read.csv(fn)
  d <- subset(d, Experiment=='Optimized')
  
  if(!missing(querydir)) {
    # try to get the original AQL queries
    d$aql <- apply(d[,c("Group", "Problem.Space")], 1, bench_getaql, querydir=querydir)
  } else {
    d$aql <- ""
  }
  
  
  
  return(d)
}

bench_desc <- function(d) {
  
  worse <- subset(d, Baseline >= 1.0)
  better <- subset(d, Baseline < 1.0)
  
  q <- quantile(d$Baseline)
  
  return (list(worse=nrow(worse), better=nrow(better), quantile=q, 
               sumTime=sum(d$us.Iteration)))
}

bench_plot <- function(d) {
  h <- sort(d$Baseline)
  result <- barplot(h, main=paste("speedup distribution"), log="y", ylim=c(min(h),max(h) * 10 ))
  lines(x=c(0, max(result)), y=c(1.0, 1.0), col="red")
  lines(x=c(0, max(result)), y=c(0.1, 0.1), col="blue", lty=2)
  lines(x=c(0, max(result)), y=c(0.01, 0.01), col="green", lty=3)
  return (result)
}





