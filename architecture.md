API
===

/procedure/<program>
------------------

This will give the contents of the given procedure

{
  contents: String
}

/procedure/<procedure>/<innerProcedure>
-----------------

This will give the contents of the given inner procedure

/search/procedure/<procedure>
------------------

This will find a program based upon the filename given

{
  results: Vec<String>
}

/search/procedure/<procedure>/<innerProcedure>
------------------

This will find the definition of the given inner procedure in the given procedure.

{
  position: FilePosition,
  procedure: String,
  contents: String,
  arguments: Vec<ProgressArguments>
}

/search/function/<program>/<function>
------------------

This will find the definition given function in the given program

{
  position: FilePosition,
  function: String,
  contents: String,
  arguments: Vec<ProgressArguments>
}

/search/event/<program>/<control>/<event>
-----------------

This will find the definition given event on the given element.

{
  position: FilePosition,
  event: String,
  control: String,
  contents: String
}

/analysisSections/<program>
------------------

This will give a vector of sections in the given program

{
  results: Vec<PReprocessorAnalysisSection>
}

