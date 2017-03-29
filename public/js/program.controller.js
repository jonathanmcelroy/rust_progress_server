(function() {
  angular.module('progressServer').controller('programController', ['$state', '$resource', function(state, resource) {
    var programController = this;
    programController.value = "Loading program...";
    programController.sections = []
    programController.createWindowSection;

    programController.toggle = section => {
      console.log(section);
      section.open = !section.open;
    }

    resource("/api/procedure/" + encodeURIComponent(state.params.name)).get(function(res) {
      programController.name = state.params.name;
      programController.fileReferences = res.file_references;
      programController.sections = res.sections.map(section => {
        section.open = false;
        return section;
      });
      programController.sections.forEach(section => {
        if (section.type == "CreateWindow") {
          programController.createWindowSection = section
        }
      });
    }).$promise.catch(function(a) {
      programController.sections = "Could not fetch program";
    });

  }]);
}());
