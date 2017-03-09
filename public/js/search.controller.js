(function() {
  angular.module('progressServer').controller('searchController', ['$state', '$resource', function(state, resource) {
    var searchController = this;

    searchController.search = state.params.contents;
    searchController.results = [];

    resource('api/search/procedure/' + searchController.search).get(function(res) {
      searchController.results = res.results;
    });

  }]);
}());
