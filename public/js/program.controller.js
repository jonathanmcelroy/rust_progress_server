(function() {
  angular.module('progressServer').controller('programController', ['$state', '$resource', function(state, resource) {
    var programController = this;

    resource("/api/program/" + encodeURIComponent(state.params.name)).get(function(res) {
      programController.name = state.params.name;
      programController.fileReferences = res.file_references;
      programController.value = res.contents;
    }).$promise.catch(function(a) {
      programController.value = "Could not fetch program";
    });

    programController.value = "Loading program...";
  }]);
}());
