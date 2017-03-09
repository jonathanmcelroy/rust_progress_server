(function() {
  angular.module('progressServer').controller('searchInnerProcedureController', ['$state', '$resource', function(state, resource) {
    var searchController = this;

    searchController.fileName = state.params.fileName;
    searchController.innerName = state.params.innerName;
    searchController.results = [];

    resource('api/search/procedure/' + searchController.fileName + '/' + searchController.innerName).get(function(res) {
      searchController.results = res.results.map(([procedure, innerProcedure]) => ({ procedure, innerProcedure }));
    });

  }]);
}());
