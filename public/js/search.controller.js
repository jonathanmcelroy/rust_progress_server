(function() {
  angular.module('progressServer').controller('searchController', ['$state', '$resource', function(state, resource) {
    var searchController = this;

    searchController.results = [];

    resource('api/search/procedure/' + state.params.contents).get(function(res) {
      searchController.results = res.results;
    });

  }]);
}());
